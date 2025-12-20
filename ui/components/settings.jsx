import { cn } from "@/utils/cn"

export function SettingsItem({ title, desc, className, children }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="font-semibold">{title}</span>
      {desc && <span className="text-xs text-secondary">{desc}</span>}
      <div className={cn("flex mt-0.5", className)}>{children}</div>
    </div>
  )
}
