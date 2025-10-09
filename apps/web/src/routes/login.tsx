import { createFileRoute } from '@tanstack/solid-router'

export const Route = createFileRoute('/login')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/login"!</div>
}
