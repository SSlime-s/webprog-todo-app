import { Client } from 'apis'

export const deleteLogout = (client: Client) => async () => {
  const res = await fetch(`${client.baseURL}/logout`, {
    credentials: 'include',
    method: 'DELETE',
  })
  return res
}
