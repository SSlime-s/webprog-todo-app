import { Client } from '..'

export const getMe = (client: Client) => async () => {
  const res = await fetch(`${client.baseURL}/me`, {
    credentials: 'include',
  })
  const data = await res.json()
  return data
}
