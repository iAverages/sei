import { createFileRoute } from '@tanstack/solid-router'
export const Route = createFileRoute('/_app/about')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <main>
      <h1>About</h1>
    </main>
  )
}
