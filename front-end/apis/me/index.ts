import { userSchema } from 'apis/parser/user'
import { Client } from '..'

export const getMe = (client: Client) => async () => {
  const res = await fetch(`${client.baseURL}/me`, {
    credentials: 'include',
  })

  if (!res.ok) {
    throw new Error(res.statusText)
  }

  const data = await res.json()
  const parsedData = userSchema.parse(data)
  return parsedData
}

export const patchMe =
  (client: Client) =>
  async (
    currentPassword: string,
    query: {
      username?: string
      display_name?: string
      password?: string
    }
  ) => {
    const res = await fetch(`${client.baseURL}/me`, {
      method: 'PATCH',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        current_password: currentPassword,
        ...query,
      }),
    })
    return res
  }

export const deleteMe = (client: Client) => async (currentPassword: string) => {
  const res = await fetch(`${client.baseURL}/me`, {
    method: 'DELETE',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      password: currentPassword,
    }),
  })
  return res
}
