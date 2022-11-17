import { Client } from '..'

export interface postSignUpReq {
  username: string
  display_name: string
  password: string
}
export const postSignUp = (client: Client) => async (req: postSignUpReq) => {
  const res = await fetch(`${client.baseURL}/signup`, {
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
