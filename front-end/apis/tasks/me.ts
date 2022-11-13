import { Client } from '..'

export interface GetTasksMeRequest {
  readonly limit?: number
  readonly offset?: number
}
export const getTasksMe =
  (client: Client) => async (req: GetTasksMeRequest) => {
    const url = new URL(`${client.baseURL}/tasks/me`)
    if (req.limit !== undefined) {
      url.searchParams.append('limit', req.limit.toString())
    }
    if (req.offset !== undefined) {
      url.searchParams.append('offset', req.offset.toString())
    }
    const res = await fetch(url, {
      credentials: 'include',
    })
    const data = await res.json()
    return data
  }
