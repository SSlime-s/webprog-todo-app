import { taskSchema } from 'apis/parser/task'
import { z } from 'zod'
import { Client } from '..'

export interface GetTasksMeRequest {
  readonly phrase?: string

  readonly limit?: number
  readonly offset?: number

  readonly state_filter?: ('icebox' | 'todo' | 'in-progress' | 'done')[]
}
export const getTasksMe =
  (client: Client) => async (req: GetTasksMeRequest) => {
    const url = new URL(`${client.baseURL}/tasks/me`)
    if (req.phrase !== undefined && req.phrase.trim() !== '') {
      url.searchParams.set('phrase', req.phrase)
    }
    if (req.limit !== undefined) {
      url.searchParams.append('limit', req.limit.toString())
    }
    if (req.offset !== undefined) {
      url.searchParams.append('offset', req.offset.toString())
    }
    if (req.state_filter !== undefined) {
      url.searchParams.append('state_filter', `[${req.state_filter.join(',')}]`)
    }
    const res = await fetch(url, {
      credentials: 'include',
    })

    if (!res.ok) {
      throw new Error(res.statusText)
    }

    const data = await res.json()
    const parsedData = z
      .object({
        items: z.array(taskSchema),
        total: z.number(),
      })
      .parse(data)
    return parsedData
  }
