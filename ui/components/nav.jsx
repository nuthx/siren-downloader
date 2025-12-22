import { Link, useLocation } from "react-router-dom"
import { cn } from "../utils/cn"

export function Nav({ children }) {
  return (
    <nav className="flex items-center justify-between gap-8 mx-10 py-8 border-b">
      <img src="/logo/siren.svg" alt="logo" className="w-32" draggable="false" />
      <div className="flex-center gap-12">
        {children}
      </div>
    </nav>
  )
}

export function NavButton({ path, title }) {
  const pathname = useLocation().pathname
  const isSelected = pathname === path || pathname.startsWith(`${path}/`)

  return (
    <Link to={path} className={cn("shrink-0 font-semibold text-base transition", isSelected ? "text-primary" : "text-secondary hover:text-primary")}>
      {title}
    </Link>
  )
}
