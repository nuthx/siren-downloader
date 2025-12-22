import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/react"
import { cn } from "@/utils/cn"
import { EllipsisVertical } from "lucide-react"

export function DropdownMenu({ items }) {
  return (
    <Menu>
      <MenuButton className="px-1 py-4 text-primary/70 hover:text-primary data-active:text-primary cursor-pointer">
        <EllipsisVertical className="size-4" />
      </MenuButton>

      <MenuItems
        className="z-50 min-w-38 bg-background border shadow-lg shadow-background transition origin-top-right data-closed:scale-95 data-closed:opacity-0"
        anchor="bottom end"
        transition
      >
        {items.map((item, index) => (
          <MenuItem key={index}>
            {() => (
              <button
                className={cn(
                  "flex flex-col items-start gap-1 w-full min-h-10 px-4 py-3 hover:text-primary/70 transition",
                  item.disabled ? "text-muted! cursor-not-allowed" : "cursor-pointer"
                )}
                disabled={item.disabled}
                onClick={item.onClick}
              >
                {item.label}
              </button>
            )}
          </MenuItem>
        ))}
      </MenuItems>
    </Menu>
  )
}
