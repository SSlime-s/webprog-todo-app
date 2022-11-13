import NextLink from 'next/link'
import { NextPage } from 'next'
import { getHello, useClient } from '../apis'
import useSWR from 'swr'
import { Card, message, Button } from 'antd'
import { useCallback } from 'react'
import { deleteLogout } from './logout'

const Home: NextPage = () => {
  const client = useClient()
  const { data } = useSWR('hello', getHello(client))

  const logout = useCallback(async () => {
    try {
      await deleteLogout(client)()
    } catch (e: any) {
      // message.error(e)
      console.error(e)
    }
  }, [])

  return (
    <div>
      <h1>Home</h1>
      <p>{data}</p>
      <Button onClick={logout}>ログアウト</Button>
      <Card>
        <ul>
          <li>
            <NextLink href="/signup">Sign Up</NextLink>
          </li>
          <li>
            <NextLink href="/login">Login</NextLink>
          </li>
        </ul>
      </Card>
    </div>
  )
}

export default Home
