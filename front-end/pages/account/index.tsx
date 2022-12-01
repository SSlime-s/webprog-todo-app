import styled from '@emotion/styled'
import { Button, Card, Form, Input, Modal } from 'antd'
import { NextPage } from 'next'
import { useRouter } from 'next/router'
import React, { useCallback, useEffect, useState } from 'react'
import { useMe } from 'usecase/user'
import NextLink from 'next/link'

const Account: NextPage = () => {
  const { data, mutate, isLoading, isUnauthorized } = useMe()
  const [form] = Form.useForm<{
    username: string
    displayName: string
    password: string
  }>()
  const [passwordForm] = Form.useForm<{
    currentPassword: string
    newPassword: string
  }>()

  const [isOpen, setIsOpen] = useState(false)
  const open = useCallback(() => {
    setIsOpen(true)
  }, [])
  const close = useCallback(() => {
    setIsOpen(false)
  }, [])

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
      <h1>
        <NextLink href="/">Account</NextLink>
      </h1>
      <Wrap>
        <Card title="プロフィール編集">
          <Form
            form={form}
            initialValues={{
              username: data?.username,
              displayName: data?.display_name,
            }}
            layout="vertical"
            onFinish={async (values) => {
              await mutate.updateMe(values.password, {
                username: values.username || undefined,
                displayName: values.displayName || undefined,
              })
            }}
          >
            <Form.Item label="ユーザー名" name="username">
              <Input />
            </Form.Item>
            <Form.Item label="表示名" name="displayName">
              <Input />
            </Form.Item>
            <Form.Item label="現在のパスワード" name="password">
              <Input.Password />
            </Form.Item>
            <Form.Item>
              <Button type="primary" htmlType="submit">
                更新
              </Button>
            </Form.Item>
          </Form>
        </Card>
        <Card title="パスワード変更">
          <Form
            form={passwordForm}
            layout="vertical"
            onFinish={async (values) => {
              await mutate.updateMe(values.currentPassword, {
                password: values.newPassword,
              })
            }}
          >
            <Form.Item label="現在のパスワード" name="currentPassword">
              <Input.Password />
            </Form.Item>
            <Form.Item label="新しいパスワード" name="newPassword">
              <Input.Password type="newPassword" />
            </Form.Item>
            <Form.Item>
              <Button type="primary" htmlType="submit">
                更新
              </Button>
            </Form.Item>
          </Form>
        </Card>
        <ButtonsWrap>
          <Button type="primary" danger onClick={open}>
            アカウント削除
          </Button>
          <Button onClick={mutate.logout}>ログアウト</Button>
        </ButtonsWrap>
      </Wrap>

      <DeleteConfirmModal
        isOpen={isOpen}
        close={close}
        deleteAccount={mutate.deleteMe}
      />
    </div>
  )
}
export default Account

const Wrap = styled.div`
  display: flex;
  flex-direction: column;
  max-width: 600px;
  margin: 0 auto;
  gap: 16px;
`
const ButtonsWrap = styled.div`
  display: flex;
  gap: 16px;
  justify-content: flex-end;
`

interface DeleteConfirmModalProps {
  isOpen: boolean
  close: () => void
  deleteAccount: (password: string) => void
}
const DeleteConfirmModal: React.FC<DeleteConfirmModalProps> = ({
  isOpen,
  close,
  deleteAccount,
}) => {
  const [password, setPassword] = useState('')

  useEffect(() => {
    if (isOpen) {
      setPassword('')
    }
  }, [isOpen])

  return (
    <Modal
      title="アカウント削除"
      visible={isOpen}
      onOk={() => {
        deleteAccount(password)
        close()
      }}
      okType="danger"
      okText="削除"
      onCancel={close}
    >
      <p>本当に削除しますか？</p>
      <p>この操作は取り消せません。</p>
      <p>確認のため、現在のパスワードを入力してください。</p>
      <Input.Password
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />
    </Modal>
  )
}
