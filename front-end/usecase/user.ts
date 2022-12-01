import { useClient } from 'apis'
import { deleteLogout } from 'apis/logout'
import { getMe, patchMe, deleteMe as deleteMeApi } from 'apis/me'
import { useCallback } from 'react'
import useSWR from 'swr'

export const useMe = () => {
  const client = useClient()
  const { data, error, mutate } = useSWR('/me', getMe(client))

  const isLoading = data === undefined && !error
  const isUnauthorized = error?.message === 'Unauthorized'

  const updateMe = useCallback(
    async (
      currentPassword: string,
      query: {
        username?: string
        displayName?: string
        password?: string
      }
    ) => {
      await patchMe(client)(currentPassword, {
        ...query,
        display_name: query.displayName,
      })
      mutate((data) => {
        if (data === undefined) {
          return undefined
        }

        return {
          ...data,
          ...query,
        }
      })
    },
    [client, mutate]
  )

  const deleteMe = useCallback(
    async (currentPassword: string) => {
      await deleteMeApi(client)(currentPassword)
      mutate()
    },
    [client, mutate]
  )

  const logout = useCallback(async () => {
    await deleteLogout(client)()
    mutate()
  }, [client, mutate])

  return {
    data,
    mutate: {
      updateMe,
      deleteMe,
      logout,
    },
    isLoading,
    isUnauthorized,
  }
}
