import { ReactNode, useRef, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [scrollState, setScrollState] = useState({
    hasScrolledFromTop: false,
    canScrollDown: false
  });

  useEffect(() => {
    const scrollElement = scrollRef.current;
    if (!scrollElement) return;

    const handleScroll = () => {
      const { scrollTop, scrollHeight, clientHeight } = scrollElement;
      const hasScrolledFromTop = scrollTop > 0;
      const canScrollDown = scrollTop < scrollHeight - clientHeight - 1;

      setScrollState({ hasScrolledFromTop, canScrollDown });
    };

    // Initial check
    handleScroll();

    scrollElement.addEventListener('scroll', handleScroll);
    const resizeObserver = new ResizeObserver(handleScroll);
    resizeObserver.observe(scrollElement);

    return () => {
      scrollElement.removeEventListener('scroll', handleScroll);
      resizeObserver.disconnect();
    };
  }, []);

  const scrollClasses = [
    scrollState.hasScrolledFromTop ? 'scroll-top-shadow' : '',
    scrollState.canScrollDown ? 'scroll-bottom-shadow' : ''
  ].filter(Boolean).join(' ');

  return (
    <div className="h-full flex flex-col">
      {/* Scrollable content area */}
      <div 
        ref={scrollRef}
        className={`flex-1 overflow-auto ${scrollClasses}`}
      >
        {children}
      </div>
      
      {/* Footer */}
      <div className="flex items-center justify-between p-3 min-h-[60px]">
        <div className="flex items-center space-x-3">
          <Badge variant="secondary" className="flex items-center space-x-2 px-3 py-2">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
            <span className="text-sm font-medium">Online</span>
          </Badge>
        </div>
        
        <div className="flex items-center space-x-2">
          <Button variant="outline" size="sm">Outline</Button>
          <Button size="sm">Compact Button</Button>
        </div>
      </div>
    </div>
  );
}