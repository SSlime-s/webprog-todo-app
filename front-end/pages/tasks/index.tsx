import {
  Button,
  Card,
  Checkbox,
  DatePicker,
  Form,
  Input,
  Modal,
  Pagination,
  Select,
} from 'antd'
import { Task } from 'apis/parser/task'
import { NextPage } from 'next'
import { useRouter } from 'next/router'
import React, { useCallback, useEffect, useMemo, useState } from 'react'
import {
  DeleteOutlined,
  EditOutlined,
  FlagFilled,
  LeftOutlined,
  PlusOutlined,
  RightOutlined,
  SearchOutlined,
} from '@ant-design/icons'
import NextLink from 'next/link'

import { PAGE_COUNT, useTasks } from 'usecase/tasks'
import { unreachable } from 'utils/typeUtils'
import { green, red, grey } from '@ant-design/colors'
import styled from '@emotion/styled'
import { DATE_FORMAT } from 'apis/parser'
import { useMe } from 'usecase/user'
import moment from 'moment'
import dayjs from 'dayjs'
import { UpdateTaskRequest } from 'apis/tasks/[id]'

const STATES = ['icebox', 'todo', 'in-progress', 'done'] as const
const isValidState = (state: string): state is typeof STATES[number] =>
  (STATES as readonly string[]).includes(state)

const TasksPage: NextPage = () => {
  const { query } = useRouter()
  const { isUnauthorized } = useMe()
  const page = useMemo(() => {
    const pageNum = Number(query.page)
    return Number.isNaN(pageNum) ? 1 : pageNum
  }, [query])
  const stateFilter = useMemo(() => {
    const state = query.state
    if (typeof state === 'string') {
      return isValidState(state) ? [state] : undefined
    }
    return state?.filter(isValidState) || undefined
  }, [query])

  const [searchText, setSearchText] = useState('')

  const { isLoading, data, mutate } = useTasks(page, searchText, stateFilter)

  const [isOpen, setIsOpen] = useState(false)
  const open = useCallback(() => setIsOpen(true), [])
  const close = useCallback(() => setIsOpen(false), [])
  const handleCreate = useCallback(
    (task: Task) => {
      void mutate.addTask(
        task.title,
        task.description,
        task.state,
        task.priority ?? undefined,
        task.due_date?.format(DATE_FORMAT) ?? undefined
      )
    },
    [mutate]
  )

  const router = useRouter()
  const onPaginationChange = useCallback(
    (page: number) => {
      router.push({ query: { page, state: stateFilter } })
    },
    [router, stateFilter]
  )

  if (isUnauthorized) {
    router.push({
      pathname: '/login',
      query: { redirect: router.asPath },
    })
    return null
  }

  return (
    <div>
      <h1>
        <NextLink href="/">Tasks</NextLink>
      </h1>
      <Wrap>
        <TopRow>
          <Input
            prefix={<SearchOutlined />}
            placeholder="タイトル"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
          />
          <Button type="primary" icon={<PlusOutlined />} onClick={open}>
            NEW
          </Button>
        </TopRow>
        <NewTaskModal
          isOpen={isOpen}
          close={close}
          handleCreate={handleCreate}
        />

        <Checkbox
          indeterminate={stateFilter?.length !== STATES.length && !!stateFilter}
          checked={stateFilter?.length === STATES.length || !stateFilter}
          onChange={(e) => {
            if (e.target.checked) {
              router.push({ query: { page, state: STATES } })
            } else {
              router.push({ query: { page } })
            }
          }}
        >
          ALL
        </Checkbox>
        <Checkbox.Group
          options={STATES.map((state) => ({
            label: state.toUpperCase(),
            value: state,
          }))}
          value={stateFilter}
          onChange={(states) => {
            router.replace({
              query: { page, state: states.map((v) => v.toString()) },
            })
          }}
          style={{ userSelect: 'none' }}
        />

        {isLoading ? (
          <div>Loading...</div>
        ) : (
          <>
            {data?.items?.map((task) => (
              <TaskCard key={task.id} task={task} mutate={mutate} />
            ))}
          </>
        )}
        <Pagination
          style={{
            alignSelf: 'center',
          }}
          current={page}
          total={data?.total ?? 0}
          pageSize={PAGE_COUNT}
          onChange={onPaginationChange}
        />
      </Wrap>
    </div>
  )
}
export default TasksPage
const Wrap = styled.div`
  display: flex;
  flex-direction: column;
  max-width: 600px;
  gap: 16px;
  margin: 0 auto;
  padding-bottom: 60px;
`

const TopRow = styled.div`
  display: flex;
  gap: 8px;
`

interface TaskCardProps {
  task: Task
  mutate: ReturnType<typeof useTasks>['mutate']
}
const TaskCard: React.FC<TaskCardProps> = ({ task, mutate }) => {
  const [isOpen, setIsOpen] = useState(false)
  const close = useCallback(() => setIsOpen(false), [])
  const handleUpdate = useCallback(
    (
      values: Omit<UpdateTaskRequest, 'due_date'> & {
        dueDate?: dayjs.Dayjs | null
      }
    ) => {
      mutate.updateTask(task.id, values)
    },
    [mutate, task.id]
  )

  const updatePriority = useCallback(
    (priority: Task['priority']) => {
      void mutate.updateTask(task.id, { priority })
    },
    [mutate, task.id]
  )
  const handleDelete = useCallback(() => {
    Modal.confirm({
      title: '本当に削除しますか？',
      content: 'この操作は取り消せません。',
      okText: '削除',
      okType: 'danger',
      cancelText: 'キャンセル',
      onOk: () => {
        void mutate.deleteTask(task.id)
      },
    })
  }, [mutate, task.id])

  const EditButton = useCallback(
    () => <EditOutlined onClick={() => setIsOpen(true)} />,
    []
  )
  const DeleteButton = useCallback(
    () => <DeleteOutlinedColored onClick={handleDelete} title="削除" />,
    [handleDelete]
  )

  const hasNextState = useMemo(() => {
    const index = STATES.indexOf(task.state)
    return index !== STATES.length - 1
  }, [task.state])
  const hasPrevState = useMemo(() => {
    const index = STATES.indexOf(task.state)
    return index !== 0
  }, [task.state])
  const changeStateNext = useCallback(() => {
    if (!hasNextState) return
    const index = STATES.indexOf(task.state)
    if (index === -1) return
    const nextState = STATES[index + 1]
    if (nextState) {
      void mutate.updateTask(task.id, { state: nextState })
    }
  }, [hasNextState, mutate, task.id, task.state])
  const changeStatePrev = useCallback(() => {
    if (!hasPrevState) return
    const index = STATES.indexOf(task.state)
    if (index === -1) return
    const prevState = STATES[index - 1]
    if (prevState) {
      void mutate.updateTask(task.id, { state: prevState })
    }
  }, [hasPrevState, mutate, task.id, task.state])

  const isEmpty = useMemo(() => {
    return (
      task.description === '' &&
      task.priority === null &&
      task.due_date === null
    )
  }, [task.description, task.due_date, task.priority])

  const dueDateState = useMemo(() => {
    if (!task.due_date) return null
    const now = dayjs(new Date())
    const dueDate = task.due_date
    const diff = dueDate.diff(now, 'day')
    if (diff <= 0) return 'overdue'
    if (diff <= 3) return 'warning'
    return null
  }, [task.due_date])

  return (
    <>
      <Card
        title={<TitleWithState title={task.title} state={task.state} />}
        actions={[<EditButton key="edit" />, <DeleteButton key="delete" />]}
        bodyStyle={{
          padding: isEmpty ? '0' : undefined,
        }}
        extra={
          <>
            <Button
              type="text"
              icon={<LeftOutlined />}
              disabled={!hasPrevState}
              onClick={changeStatePrev}
            />
            <Button
              type="text"
              icon={<RightOutlined />}
              disabled={!hasNextState}
              onClick={changeStateNext}
            />
          </>
        }
      >
        <CardWrap>
          {task.description}
          {task.priority !== null ? (
            <PriorityWrap>
              <PrioritySelector
                priority={task.priority}
                onChange={updatePriority}
              />
            </PriorityWrap>
          ) : null}
          {task.due_date !== null ? (
            <DueDateWrap>
              <DatePicker
                value={moment(task.due_date.toDate())}
                onChange={(date) => {
                  if (date) {
                    void mutate.updateTask(task.id, {
                      dueDate: dayjs(date.toDate()),
                    })
                  }
                }}
                status={
                  dueDateState === 'overdue'
                    ? 'error'
                    : dueDateState === 'warning'
                    ? 'warning'
                    : undefined
                }
                bordered={dueDateState !== null}
              />
            </DueDateWrap>
          ) : null}
        </CardWrap>
      </Card>
      <TaskEditModal
        isOpen={isOpen}
        close={close}
        defaultTask={task}
        handleUpdate={handleUpdate}
      />
    </>
  )
}
const CardWrap = styled.div`
  display: flex;
  flex-direction: column;
  gap: 8px;
`
const PriorityWrap = styled.div`
  margin-right: auto;
  margin-left: -12px;
`
const DueDateWrap = styled.div`
  margin-right: auto;
`
const DeleteOutlinedColored = styled(DeleteOutlined)`
  color: ${red[5]} !important;
`

interface TaskEditModalProps {
  isOpen: boolean
  close: () => void
  defaultTask: Task
  handleUpdate: (
    task: Omit<UpdateTaskRequest, 'due_date'> & {
      dueDate?: dayjs.Dayjs | null
    }
  ) => void
}
const TaskEditModal: React.FC<TaskEditModalProps> = ({
  isOpen,
  close,
  defaultTask,
  handleUpdate,
}) => {
  const [form] = Form.useForm<
    Omit<UpdateTaskRequest, 'due_date'> & {
      dueDate?: dayjs.Dayjs | null
    }
  >()

  const [isDirty, setIsDirty] = useState(false)
  const setDirty = useCallback(() => {
    setIsDirty(true)
  }, [])

  useEffect(() => {
    if (isOpen) {
      form.resetFields()
      form.setFieldsValue(defaultTask)
      setIsDirty(false)
    }
  }, [isOpen, form, defaultTask])

  const onOk = useCallback(async () => {
    const values = await form.validateFields()
    handleUpdate({
      ...values,
      dueDate:
        values.dueDate !== undefined && values.dueDate !== null
          ? dayjs(values.dueDate.toDate())
          : null,
    })

    close()
  }, [form, handleUpdate, close])
  const onCancel = useCallback(() => {
    if (!isDirty) {
      close()
      return
    }
    Modal.confirm({
      title: '変更を破棄しますか？',
      okText: '破棄',
      okType: 'danger',
      cancelText: 'キャンセル',
      onOk: close,
    })
  }, [close, isDirty])

  return (
    <Modal
      title="タスクの編集"
      open={isOpen}
      onOk={onOk}
      onCancel={onCancel}
      okButtonProps={{
        disabled: !isDirty,
      }}
    >
      <Form
        form={form}
        initialValues={defaultTask}
        layout="vertical"
        onValuesChange={setDirty}
      >
        <Form.Item label="" name="state">
          <Select>
            <Select.Option value="icebox">
              <TitleWithState title="Icebox" state="icebox" />
            </Select.Option>
            <Select.Option value="todo">
              <TitleWithState title="Todo" state="todo" />
            </Select.Option>
            <Select.Option value="in-progress">
              <TitleWithState title="In Progress" state="in-progress" />
            </Select.Option>
            <Select.Option value="done">
              <TitleWithState title="Done" state="done" />
            </Select.Option>
          </Select>
        </Form.Item>
        <Form.Item label="タイトル" name="title">
          <Input />
        </Form.Item>
        <Form.Item label="説明" name="description">
          <Input.TextArea />
        </Form.Item>
        <Form.Item label="優先度" name="priority">
          <Select>
            <Select.Option value="high">
              <PriorityTip priority="high" />
            </Select.Option>
            <Select.Option value="medium">
              <PriorityTip priority="medium" />
            </Select.Option>
            <Select.Option value="low">
              <PriorityTip priority="low" />
            </Select.Option>
          </Select>
        </Form.Item>
        <Form.Item label="期限" name="dueDate">
          <DatePicker />
        </Form.Item>
      </Form>
    </Modal>
  )
}
interface NewTaskModalProps {
  isOpen: boolean
  close: () => void
  handleCreate: (task: Task) => void
}
const NewTaskModal: React.FC<NewTaskModalProps> = ({
  isOpen,
  close,
  handleCreate,
}) => {
  const [form] = Form.useForm<Task>()

  const onOk = useCallback(async () => {
    const values = await form.validateFields()
    handleCreate({
      ...values,
      due_date:
        values.due_date !== null ? dayjs(values.due_date.toDate()) : null,
    })
    form.resetFields()
    close()
  }, [form, handleCreate, close])
  const onCancel = useCallback(() => {
    close()
  }, [close])

  useEffect(() => {
    if (isOpen) {
      form.resetFields()
    }
  }, [isOpen, form])

  return (
    <Modal title="新規タスク" open={isOpen} onOk={onOk} onCancel={onCancel}>
      <Form
        form={form}
        initialValues={{
          state: 'todo',
          title: '',
          description: '',
          priority: null,
        }}
        layout="vertical"
      >
        <Form.Item label="" name="state">
          <Select>
            <Select.Option value="icebox">
              <TitleWithState title="Icebox" state="icebox" />
            </Select.Option>
            <Select.Option value="todo">
              <TitleWithState title="Todo" state="todo" />
            </Select.Option>
            <Select.Option value="in-progress">
              <TitleWithState title="In Progress" state="in-progress" />
            </Select.Option>
            <Select.Option value="done">
              <TitleWithState title="Done" state="done" />
            </Select.Option>
          </Select>
        </Form.Item>
        <Form.Item label="タイトル" name="title" required>
          <Input />
        </Form.Item>
        <Form.Item label="説明" name="description">
          <Input.TextArea />
        </Form.Item>
        <Form.Item label="優先度" name="priority">
          <Select>
            <Select.Option value="high">
              <PriorityTip priority="high" />
            </Select.Option>
            <Select.Option value="medium">
              <PriorityTip priority="medium" />
            </Select.Option>
            <Select.Option value="low">
              <PriorityTip priority="low" />
            </Select.Option>
          </Select>
        </Form.Item>
        <Form.Item label="期限" name="due_date">
          <DatePicker />
        </Form.Item>
      </Form>
    </Modal>
  )
}

interface TitleWithStateProps {
  title: string
  state: Task['state']
}
const TitleWithState: React.FC<TitleWithStateProps> = ({ state, title }) => {
  switch (state) {
    case 'icebox':
      return <span>🧊 {title}</span>
    case 'todo':
      return <span>📝 {title}</span>
    case 'in-progress':
      return <span>🕶️ {title}</span>
    case 'done':
      return <span>🎉 {title}</span>
    default:
      return unreachable(state)
  }
}

const priorities = ['low', 'medium', 'high'] as const
interface PrioritySelectorProps {
  priority: Task['priority']
  onChange: (priority: Task['priority']) => void
}
const PrioritySelector: React.FC<PrioritySelectorProps> = ({
  priority,
  onChange,
}) => {
  const handleChange = useCallback(
    (value: Task['priority']) => {
      onChange(value)
    },
    [onChange]
  )

  return (
    <Select value={priority} bordered={false} onChange={handleChange}>
      {priorities.map((priority) => (
        <Select.Option key={priority} value={priority}>
          <PriorityTip priority={priority} />
        </Select.Option>
      ))}
    </Select>
  )
}

const priorityColor = {
  high: red[4],
  medium: green[4],
  low: grey[4],
} as const
interface PriorityTipProps {
  priority: 'low' | 'medium' | 'high'
}
const PriorityTip: React.FC<PriorityTipProps> = ({ priority }) => {
  return (
    <div>
      <FlagFilledWithColor fill={priorityColor[priority]} />
      <PriorityText>{priority}</PriorityText>
    </div>
  )
}

const FlagFilledWithColor = styled(FlagFilled)<{ fill: string }>`
  color: ${(props) => props.fill};
`
const PriorityText = styled.span`
  margin-left: 4px;
`
