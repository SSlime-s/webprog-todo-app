import { z } from 'zod'

export const tagSchema = z.object({
  id: z.string(),
  name: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type Tag = z.infer<typeof tagSchema>

export const prioritySchema = z.union([
  z.literal('low'),
  z.literal('medium'),
  z.literal('high'),
])
export type Priority = z.infer<typeof prioritySchema>

export const stateSchema = z.union([
  z.literal('icebox'),
  z.literal('todo'),
  z.literal('in-progress'),
  z.literal('done'),
])
export type State = z.infer<typeof stateSchema>

export const taskSchema = z.object({
  id: z.string(),
  author_id: z.string(),
  title: z.string(),
  description: z.string(),
  state: stateSchema,
  priority: prioritySchema.optional(),
  due_date: z.string().optional(),

  created_at: z.string(),
  updated_at: z.string(),
})
export type Task = z.infer<typeof taskSchema>
