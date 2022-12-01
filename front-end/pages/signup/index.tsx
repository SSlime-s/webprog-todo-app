import { NextPage } from 'next'
import { Form, Input, Button, Card, message } from 'antd'
import styled from '@emotion/styled'
import { postSignUp } from 'apis/signup'
import { useClient } from 'apis'
import { useCallback, useMemo } from 'react'
import { useRouter } from 'next/router'
import NextLink from 'next/link'
import { UrlObject } from 'url'

const Container = styled.div`
  display: grid;
  place-items: center;
  min-height: 100vh;
  background-color: #ced7e0;
`

const FormCard = styled(Card)``

const FormWrap = styled(Form)`
  display: flex;
  flex-direction: column;
`

interface FormValues {
  username: string
  password: string
}

const SignUp: NextPage = () => {
  const client = useClient()
  const [form] = Form.useForm<FormValues>()

  const router = useRouter()
  const { redirect } = router.query

  const loginPath = useMemo(() => {
    const base: UrlObject = {
      pathname: '/login',
    }
    if (typeof redirect === 'string') {
      base.query = { redirect }
    }
    return base
  }, [redirect])
  const redirectTo = useMemo(() => {
    if (typeof redirect === 'string') {
      return redirect
    }
    return '/'
  }, [redirect])

  const submit = useCallback(async () => {
    const values = await form.validateFields()
    try {
      const res = await postSignUp(client)({
        display_name: values.username,
        ...values,
      })
      if (res.status === 200) {
        router.push(redirectTo)
      } else {
        const text = await res.text()
        message.error(text)
      }
    } catch (e: any) {
      message.error(e)
      console.log(e)
    }
  }, [client, form, redirectTo, router])

  return (
    <Container>
      <FormCard>
        <FormWrap labelCol={{ span: 8 }} wrapperCol={{ span: 16 }} form={form}>
          <Form.Item label="username" name="username" required>
            <Input />
          </Form.Item>
          <Form.Item label="password" name="password" required>
            <Input.Password />
          </Form.Item>

          <Form.Item wrapperCol={{ offset: 8, span: 16 }}>
            <Button type="primary" htmlType="submit" onClick={submit}>
              登録
            </Button>
          </Form.Item>
        </FormWrap>

        <p>
          すでにアカウントがある方は
          <NextLink href={loginPath} className="ant-btn-link">
            こちら
          </NextLink>
        </p>
      </FormCard>
    </Container>
  )
}

export default SignUp
