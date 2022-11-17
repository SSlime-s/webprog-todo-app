import { Client } from '..'

export interface postLoginReq {
  username: string
  password: string
}
export const postLogin = (client: Client) => async (req: postLoginReq) => {
  const res = await fetch(`${client.baseURL}/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(req),
    credentials: 'include',
  })

  if (!res.ok) {
    throw new Error(res.statusText)
  }

  return res
}
