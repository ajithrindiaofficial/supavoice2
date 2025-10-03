import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";

export function ComponentShowcase() {
  const [switchValue, setSwitchValue] = useState(false);
  const [checkboxValue, setCheckboxValue] = useState(false);
  const [radioValue, setRadioValue] = useState("option1");

  return (
    <div className="space-y-4 p-3">
      <Tabs defaultValue="forms" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="forms">Forms</TabsTrigger>
          <TabsTrigger value="display">Display</TabsTrigger>
          <TabsTrigger value="navigation">Navigation</TabsTrigger>
          <TabsTrigger value="feedback">Feedback</TabsTrigger>
        </TabsList>

        <TabsContent value="forms" className="space-y-6">
          <Card>
            <CardHeader className="p-4">
              <CardTitle className="text-base">Form Controls</CardTitle>
              <CardDescription>Various input components for user interaction</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 p-4 pt-0">
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input id="email" placeholder="Enter your email" type="email" className="h-8 py-1 px-2 text-sm" />
              </div>

              <div className="space-y-2">
                <Label htmlFor="message">Message</Label>
                <Textarea id="message" placeholder="Type your message here" className="min-h-[60px] py-1 px-2 text-sm" />
              </div>

              <div className="space-y-2">
                <Label htmlFor="country">Country</Label>
                <Select>
                  <SelectTrigger className="h-8 px-2 text-sm">
                    <SelectValue placeholder="Select a country" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="us">United States</SelectItem>
                    <SelectItem value="uk">United Kingdom</SelectItem>
                    <SelectItem value="ca">Canada</SelectItem>
                    <SelectItem value="au">Australia</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center space-x-2">
                  <Checkbox 
                    id="terms" 
                    checked={checkboxValue}
                    onCheckedChange={(checked) => setCheckboxValue(checked === true)}
                  />
                  <Label htmlFor="terms">Accept terms and conditions</Label>
                </div>

                <div className="flex items-center space-x-2">
                  <Switch 
                    id="notifications" 
                    checked={switchValue}
                    onCheckedChange={setSwitchValue}
                  />
                  <Label htmlFor="notifications">Enable notifications</Label>
                </div>
              </div>

              <div className="space-y-2">
                <Label>Preferred contact method</Label>
                <RadioGroup value={radioValue} onValueChange={setRadioValue}>
                  <div className="flex items-center space-x-2">
                    <RadioGroupItem value="email" id="radio-email" />
                    <Label htmlFor="radio-email">Email</Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <RadioGroupItem value="phone" id="radio-phone" />
                    <Label htmlFor="radio-phone">Phone</Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <RadioGroupItem value="sms" id="radio-sms" />
                    <Label htmlFor="radio-sms">SMS</Label>
                  </div>
                </RadioGroup>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="display" className="space-y-6">
          <Card>
            <CardHeader className="p-4">
              <CardTitle className="text-base">Display Components</CardTitle>
              <CardDescription>Components for displaying information and content</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 p-4 pt-0">
              <div className="space-y-2">
                <Label>Badges</Label>
                <div className="flex flex-wrap gap-2">
                  <Badge variant="default">Default</Badge>
                  <Badge variant="secondary">Secondary</Badge>
                  <Badge variant="destructive">Destructive</Badge>
                  <Badge variant="outline">Outline</Badge>
                </div>
              </div>

              <Separator />

              <div className="space-y-3">
                <Label>Cards</Label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <Card>
                    <CardHeader className="p-3">
                      <CardTitle className="text-sm">Card Title</CardTitle>
                      <CardDescription className="text-xs">This is a sample card description</CardDescription>
                    </CardHeader>
                    <CardContent className="p-3 pt-0">
                      <p className="text-xs">Card content goes here. This is an example of how content appears within a card component.</p>
                    </CardContent>
                  </Card>
                  <Card>
                    <CardHeader className="p-3">
                      <CardTitle className="text-sm">Another Card</CardTitle>
                      <CardDescription className="text-xs">Different content in this card</CardDescription>
                    </CardHeader>
                    <CardContent className="p-3 pt-0">
                      <p className="text-xs">Each card can contain different types of content and layouts.</p>
                    </CardContent>
                  </Card>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="navigation" className="space-y-6">
          <Card>
            <CardHeader className="p-4">
              <CardTitle className="text-base">Navigation Components</CardTitle>
              <CardDescription>Components for navigation and organization</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 p-4 pt-0">
              <div className="space-y-2">
                <Label>Tabs (Nested Example)</Label>
                <Tabs defaultValue="tab1" className="w-full">
                  <TabsList>
                    <TabsTrigger value="tab1">Tab 1</TabsTrigger>
                    <TabsTrigger value="tab2">Tab 2</TabsTrigger>
                    <TabsTrigger value="tab3">Tab 3</TabsTrigger>
                  </TabsList>
                  <TabsContent value="tab1" className="space-y-2">
                    <p className="text-sm">Content for the first tab. This demonstrates how tabs can be used for organizing content.</p>
                  </TabsContent>
                  <TabsContent value="tab2" className="space-y-2">
                    <p className="text-sm">Content for the second tab. Each tab can have different content and layouts.</p>
                  </TabsContent>
                  <TabsContent value="tab3" className="space-y-2">
                    <p className="text-sm">Content for the third tab. Tabs are great for space-efficient content organization.</p>
                  </TabsContent>
                </Tabs>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="feedback" className="space-y-6">
          <Card>
            <CardHeader className="p-4">
              <CardTitle className="text-base">Feedback & Actions</CardTitle>
              <CardDescription>Interactive components for user actions and feedback</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 p-4 pt-0">
              <div className="space-y-2">
                <Label>Button Variants</Label>
                <div className="flex flex-wrap gap-2">
                  <Button variant="default" size="sm">Default</Button>
                  <Button variant="secondary" size="sm">Secondary</Button>
                  <Button variant="destructive" size="sm">Destructive</Button>
                  <Button variant="outline" size="sm">Outline</Button>
                  <Button variant="ghost" size="sm">Ghost</Button>
                  <Button variant="link" size="sm">Link</Button>
                </div>
              </div>

              <Separator />

              <div className="space-y-2">
                <Label>Button Sizes</Label>
                <div className="flex items-center gap-2">
                  <Button size="sm">Small</Button>
                  <Button size="default">Default</Button>
                  <Button size="lg">Large</Button>
                </div>
              </div>

              <Separator />

              <div className="space-y-2">
                <Label>Interactive States</Label>
                <div className="space-y-2">
                  <p className="text-xs text-muted-foreground">
                    Current switch state: <Badge variant={switchValue ? "default" : "secondary"} className="text-xs px-2 py-0">{switchValue ? "On" : "Off"}</Badge>
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Current checkbox state: <Badge variant={checkboxValue ? "default" : "secondary"} className="text-xs px-2 py-0">{checkboxValue ? "Checked" : "Unchecked"}</Badge>
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Current radio selection: <Badge variant="outline" className="text-xs px-2 py-0">{radioValue}</Badge>
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}