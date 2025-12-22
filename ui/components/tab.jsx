import { Fragment } from "react"
import { cn } from "@/utils/cn"

export function Tab({ items, value, onChange, className }) {
  return (
    <div className={cn("flex items-center gap-5", className)}>
      {items.map((item, index) => (
        <Fragment key={item.value}>
          {index > 0 && <div className="w-px h-4 bg-secondary rotate-16" />}
          <button
            onClick={() => onChange(item.value)}
            className={cn("font-semibold cursor-pointer transition", value === item.value ? "text-primary" : "text-secondary hover:text-primary")}
          >
            {item.label}
          </button>
        </Fragment>
      ))}
    </div>
  )
}
