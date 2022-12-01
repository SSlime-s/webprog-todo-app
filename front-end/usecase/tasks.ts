import { useClient } from 'apis'
import { postTask } from 'apis/tasks'
import {
  updateTask as updateTaskApi,
  deleteTask as deleteTaskApi,
} from 'apis/tasks/[id]'
import { getTasksMe } from 'apis/tasks/me'
import { useCallback, useMemo } from 'react'
import useSWR from 'swr'
import dayjs from 'dayjs'

export const PAGE_COUNT = 10
export const useTasks = (
  page: number,
  phrase?: string,
  stateFilter?: ('icebox' | 'todo' | 'in-progress' | 'done')[]
) => {
  const client = useClient()
  const { data, error, mutate } = useSWR(
    ['/tasks/me', page, phrase, stateFilter],
    (_, page, phrase, stateFilter) =>
      getTasksMe(client)({
        phrase,
        limit: PAGE_COUNT,
        offset: PAGE_COUNT * (page - 1),
        state_filter: stateFilter || undefined,
      })
  )
  const taskData = useMemo(() => {
    if (data === undefined) {
      return undefined
    }

    return {
      ...data,
      totalPage: Math.ceil(data.total / PAGE_COUNT),
    }
  }, [data])

  const isLoading = data === undefined && !error

  const addTask = useCallback(
    async (
      title: string,
      description: string,
      state: 'icebox' | 'todo' | 'in-progress' | 'done',
      priority?: 'low' | 'medium' | 'high',
      dueDate?: string
    ) => {
      await postTask(client)({
        title,
        description,
        state,
        priority,
        due_date: dueDate,
      })
      mutate()
    },
    [client, mutate]
  )

  const updateTask = useCallback(
    async (
      id: string,
      query: {
        title?: string
        description?: string
        state?: 'icebox' | 'todo' | 'in-progress' | 'done'
        priority?: 'low' | 'medium' | 'high' | null
        dueDate?: string | null
      }
    ) => {
      await updateTaskApi(client)(id, query)

      mutate((data) => {
        if (!data) {
          return data
        }

        return {
          total: data.total,
          items: data.items.map((item) => {
            if (item.id !== id) {
              return item
            }

            const due_date =
              query.dueDate === null ? undefined : dayjs(query.dueDate)

            return {
              ...item,
              ...query,
              priority: query.priority ?? item.priority,
              due_date: due_date ?? item.due_date,
            }
          }),
        }
      })
    },
    [client, mutate]
  )

  const deleteTask = useCallback(
    async (id: string) => {
      await deleteTaskApi(client)(id)
      mutate((data) => {
        if (!data) {
          return data
        }

        return {
          total: data.total - 1,
          items: data.items.filter((item) => item.id !== id),
        }
      })
    },
    [client, mutate]
  )

  const changeTaskState = useCallback(
    async (id: string, state: 'icebox' | 'todo' | 'in-progress' | 'done') => {
      await updateTask(id, { state })

      mutate((data) => {
        if (!data) {
          return data
        }

        return {
          total: data.total,
          items: data.items.map((item) => {
            if (item.id !== id) {
              return item
            }

            return {
              ...item,
              state,
            }
          }),
        }
      })
    },
    [updateTask, mutate]
  )

  return {
    isLoading,
    data: taskData,
    mutate: {
      addTask,
      updateTask,
      deleteTask,
      changeTaskState,
    },
  }
}
