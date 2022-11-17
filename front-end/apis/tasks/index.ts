import { Client } from '..'

export interface PostTaskRequest {
  readonly title: string
  readonly description: string
  readonly state: 'icebox' | 'todo' | 'in-progress' | 'done'
  readonly priority?: 'low' | 'medium' | 'high'
  readonly due_date?: string
}
export const postTask = (client: Client) => async (task: PostTaskRequest) => {
  const res = await fetch(`${client.baseURL}/tasks`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(task),
    credentials: 'include',
  })

  if (!res.ok) {
    throw new Error(res.statusText)
  }

  return res
}
