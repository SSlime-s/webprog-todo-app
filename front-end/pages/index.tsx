import NextLink from 'next/link'
import { NextPage } from 'next'
import { useMe } from 'usecase/user'
import { useRouter } from 'next/router'

const Home: NextPage = () => {
  const { data: me, isLoading, isUnauthorized } = useMe()
  const router = useRouter()

  if (isUnauthorized) {
    router.push({
      pathname: '/login',
      query: { redirect: router.asPath },
    })
    return null
  }

  if (isLoading) {
    return <div>loading...</div>
  }

  return (
    <div>
      <h1>Home</h1>
      <div>Hello {me?.display_name || me?.username || 'anonymous'}!</div>
      <ul>
        <li>
          <NextLink href="/account">アカウント管理</NextLink>
        </li>
        <li>
          <NextLink href="tasks">タスク管理</NextLink>
        </li>
      </ul>
    </div>
  )
}

export default Home
