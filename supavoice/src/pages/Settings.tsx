import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs';
import { Button } from '../components/ui/button';

interface ModelRecord {
  id: string;
  name: string;
  kind: 'Whisper' | 'LLM';
  size_mb: number;
  download_url: string;
  checksum: string;
  status: ModelStatus;
  path?: string;
}

type ModelStatus =
  | 'NotInstalled'
  | { Downloading: { progress: number; bytes: number; total: number } }
  | 'Installed'
  | { Failed: { error: string } };

export default function Settings() {
  const [models, setModels] = useState<ModelRecord[]>([]);
  const [diskSpace, setDiskSpace] = useState<number>(0);

  useEffect(() => {
    loadModels();
    loadDiskSpace();

    // Listen for download progress events
    const progressUnlisten = listen('download_progress', (event: any) => {
      const { model_id, progress, bytes, total } = event.payload;
      setModels((prev) =>
        prev.map((m) =>
          m.id === model_id
            ? { ...m, status: { Downloading: { progress, bytes, total } } }
            : m
        )
      );
    });

    // Listen for download complete events
    const completeUnlisten = listen('download_complete', (event: any) => {
      const { model_id } = event.payload;
      setModels((prev) =>
        prev.map((m) => (m.id === model_id ? { ...m, status: 'Installed' } : m))
      );
    });

    // Listen for download failed events
    const failedUnlisten = listen('download_failed', (event: any) => {
      const { model_id, error } = event.payload;
      setModels((prev) =>
        prev.map((m) =>
          m.id === model_id ? { ...m, status: { Failed: { error } } } : m
        )
      );
    });

    return () => {
      progressUnlisten.then((fn) => fn());
      completeUnlisten.then((fn) => fn());
      failedUnlisten.then((fn) => fn());
    };
  }, []);

  const loadModels = async () => {
    try {
      const modelList = await invoke<ModelRecord[]>('list_models');
      setModels(modelList);
    } catch (error) {
      console.error('Failed to load models:', error);
    }
  };

  const loadDiskSpace = async () => {
    try {
      const space = await invoke<number>('get_disk_space');
      setDiskSpace(space);
    } catch (error) {
      console.error('Failed to get disk space:', error);
    }
  };

  const handleDownload = async (modelId: string) => {
    try {
      await invoke('start_download', { modelId });
    } catch (error) {
      console.error('Failed to start download:', error);
    }
  };

  const handleDelete = async (modelId: string) => {
    try {
      await invoke('delete_model', { modelId });
      await loadModels();
    } catch (error) {
      console.error('Failed to delete model:', error);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const getStatusText = (status: ModelStatus) => {
    if (status === 'NotInstalled') return 'Not Installed';
    if (status === 'Installed') return 'Installed';
    if (typeof status === 'object' && 'Downloading' in status) {
      return `Downloading ${status.Downloading.progress.toFixed(1)}%`;
    }
    if (typeof status === 'object' && 'Failed' in status) {
      return `Failed: ${status.Failed.error}`;
    }
    return 'Unknown';
  };

  const isDownloading = (status: ModelStatus) => {
    return typeof status === 'object' && 'Downloading' in status;
  };

  const isInstalled = (status: ModelStatus) => {
    return status === 'Installed';
  };

  return (
    <div className="p-6 h-full overflow-auto">
      <h1 className="text-2xl font-bold mb-4">Settings</h1>

      <Tabs defaultValue="models" className="w-full">
        <TabsList>
          <TabsTrigger value="models">Models</TabsTrigger>
          <TabsTrigger value="api">API Keys</TabsTrigger>
          <TabsTrigger value="preferences">Preferences</TabsTrigger>
        </TabsList>

        <TabsContent value="models" className="space-y-4">
          <div className="rounded-lg border p-4 bg-muted/50">
            <p className="text-sm text-muted-foreground">
              Free disk space: {formatBytes(diskSpace)}
            </p>
          </div>

          <div className="space-y-4">
            {models
              .filter((m) => m.kind === 'Whisper')
              .map((model) => (
                <div
                  key={model.id}
                  className="rounded-lg border p-4 flex items-center justify-between"
                >
                  <div>
                    <h3 className="font-semibold">{model.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {model.size_mb} MB · {getStatusText(model.status)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    {!isInstalled(model.status) && !isDownloading(model.status) && (
                      <Button onClick={() => handleDownload(model.id)}>
                        Download
                      </Button>
                    )}
                    {isInstalled(model.status) && (
                      <Button
                        variant="destructive"
                        onClick={() => handleDelete(model.id)}
                      >
                        Delete
                      </Button>
                    )}
                    {isDownloading(model.status) && (
                      <Button variant="secondary" disabled>
                        Downloading...
                      </Button>
                    )}
                  </div>
                </div>
              ))}
          </div>

          <h2 className="text-xl font-semibold mt-6">LLM Models</h2>
          <div className="space-y-4">
            {models
              .filter((m) => m.kind === 'LLM')
              .map((model) => (
                <div
                  key={model.id}
                  className="rounded-lg border p-4 flex items-center justify-between"
                >
                  <div>
                    <h3 className="font-semibold">{model.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {model.size_mb} MB · {getStatusText(model.status)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    {!isInstalled(model.status) && !isDownloading(model.status) && (
                      <Button onClick={() => handleDownload(model.id)}>
                        Download
                      </Button>
                    )}
                    {isInstalled(model.status) && (
                      <Button
                        variant="destructive"
                        onClick={() => handleDelete(model.id)}
                      >
                        Delete
                      </Button>
                    )}
                    {isDownloading(model.status) && (
                      <Button variant="secondary" disabled>
                        Downloading...
                      </Button>
                    )}
                  </div>
                </div>
              ))}
          </div>
        </TabsContent>

        <TabsContent value="api">
          <div className="rounded-lg border p-4">
            <p className="text-sm text-muted-foreground">
              API key configuration coming soon...
            </p>
          </div>
        </TabsContent>

        <TabsContent value="preferences">
          <div className="rounded-lg border p-4">
            <p className="text-sm text-muted-foreground">
              Preferences configuration coming soon...
            </p>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
