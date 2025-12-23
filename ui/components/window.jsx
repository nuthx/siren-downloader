export function AppWindow({ children }) {
  return (
    <div className="flex flex-col h-screen px-1 bg-[url('/background/inner.png')] bg-cover bg-center">
      {children}
    </div>
  )
}

export function MainContent({ children }) {
  return (
    <div className="flex-1 overflow-y-auto min-h-0">
      {children}
    </div>
  )
}
