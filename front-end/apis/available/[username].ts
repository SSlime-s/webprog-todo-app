import { Client } from '..'

export const getAvailable = (client: Client) => async (username: string) => {
  const res = await fetch(`${client.baseURL}/available/${username}`, {
    credentials: 'include',
  })
  const data = await res.json()
  return data
}
