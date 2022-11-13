import { useMemo } from 'react'

export interface Client {
  readonly baseURL: string
}

export const useClient = () => {
  const baseURL = 'http://localhost:8080'
  const client = useMemo(() => ({ baseURL }), [baseURL])
  return client
}

export const getHello = (client: Client) => async () => {
  const res = await fetch(`${client.baseURL}/`, {
    credentials: 'include',
  })
  const data = await res.text()
  return data
}
