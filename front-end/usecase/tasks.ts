import { useClient } from 'apis'
import { getTasksMe } from 'apis/tasks/me'
import useSWR from 'swr'

export const PAGE_COUNT = 20
export const useTasks = (page: number) => {
  const client = useClient()
  const { data, error, mutate } = useSWR(['/tasks/me', page], (_, page) =>
    getTasksMe(client)({
      limit: PAGE_COUNT,
      offset: PAGE_COUNT * (page - 1),
    })
  )

  const isLoading = !data && !error
  const isEmpty = data?.items.length === 0

  return {
    isLoading,
    isEmpty,
  }
}
