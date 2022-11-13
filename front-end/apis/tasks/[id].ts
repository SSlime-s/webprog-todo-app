import { Client } from '..'

export const getTask = (client: Client) => async (id: string) => {
  const res = await fetch(`${client.baseURL}/tasks/${id}`, {
    credentials: 'include',
  })
  const data = await res.json()
  return data
}

export const deleteTask = (client: Client) => async (id: string) => {
  const res = await fetch(`${client.baseURL}/tasks/${id}`, {
    credentials: 'include',
    method: 'DELETE',
  })
  return res
}

export interface UpdateTaskRequest {
  readonly title?: string
  readonly description?: string
  readonly state?: 'icebox' | 'todo' | 'in-progress' | 'done'
  readonly priority?: 'low' | 'medium' | 'high' | null
  readonly due_date?: string | null
}
export const updateTask =
  (client: Client) => async (id: string, req: UpdateTaskRequest) => {
    const res = await fetch(`${client.baseURL}/tasks/${id}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(req),
      credentials: 'include',
    })
    const data = await res.json()
    return data
  }
