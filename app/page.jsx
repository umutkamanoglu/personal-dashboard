import { SidebarTrigger } from "@/components/ui/sidebar";

export default function Home() {
  return (
    <div className="flex-1">
      <span className="bg-sidebar flex h-12 fixed w-full items-center px-3">
        <SidebarTrigger className="cursor-pointer" />

      </span>
    </div>
  );
}
