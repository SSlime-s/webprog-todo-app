import { Client } from '..'
import { z } from 'zod'

export const availableSchema = z.boolean()
export const getAvailable = (client: Client) => async (username: string) => {
  const res = await fetch(`${client.baseURL}/available/${username}`, {
    credentials: 'include',
  })

  if (!res.ok) {
    throw new Error(res.statusText)
  }

  const data = await res.json()
  const parsedData = availableSchema.parse(data)
  return parsedData
}
