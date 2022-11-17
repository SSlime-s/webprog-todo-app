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
