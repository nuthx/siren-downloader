import { cn } from "@/utils/cn"

export function Button({ disabled = false, children, className, ...props }) {
  return (
    <button
      className={cn(
        "flex-center gap-2 h-10 px-4 border border-secondary font-semibold backdrop-blur-2xs cursor-pointer transition shrink-0 hover:border-secondary/70 hover:text-primary/70",
        disabled && "border-muted! text-muted! cursor-not-allowed",
        className
      )}
      disabled={disabled}
      {...props}
    >
      {children}
    </button>
  )
}
