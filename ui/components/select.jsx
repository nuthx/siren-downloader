import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/react"
import { ChevronDown } from "lucide-react"
import { cn } from "@/utils/cn"

export function Select({ options, value, onChange, className }) {
  const selectedOption = options.find((opt) => opt.value === value)

  return (
    <Listbox value={value} onChange={onChange}>
      <ListboxButton className={cn("group flex-center gap-3 w-full h-10 px-4 border backdrop-blur-2xs cursor-pointer", className)}>
        <span className="flex-1 text-left truncate cursor-pointer">{selectedOption?.label || ""}</span>
        <ChevronDown className="size-4 text-secondary shrink-0 transition-all group-data-open:rotate-180" />
      </ListboxButton>

      <ListboxOptions
        anchor="bottom"
        className="w-(--button-width) mt-1 border bg-background shadow-xl shadow-background overflow-y-auto transition data-closed:-translate-y-2 data-closed:opacity-0"
        transition
      >
        {options.map((item) => (
          <ListboxOption
            key={item.value}
            value={item.value}
            className="flex flex-col items-start gap-1 min-h-10 px-4 py-3 cursor-pointer hover:text-primary/70 data-selected:bg-border/40 transition"
          >
            {item.label}
            {item?.desc && <span className="text-[10px] text-secondary">{item.desc}</span>}
          </ListboxOption>
        ))}
      </ListboxOptions>
    </Listbox>
  )
}
