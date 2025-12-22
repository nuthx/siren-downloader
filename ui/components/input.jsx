import { cn } from "@/utils/cn"

export function Input({ className, ...props }) {
  return (
    <input
      className={cn("flex-1 flex items-center h-10 px-4 border backdrop-blur-2xs placeholder:text-secondary", className)}
      {...props}
    />
  )
}
