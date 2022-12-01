import { z } from 'zod'
import dayjs from 'dayjs'

export const DATE_FORMAT = 'YYYY-MM-DD HH:mm:ss'
export const dateSchema = z
  .string()
  .refine((s) => dayjs(s, DATE_FORMAT).isValid())
  .transform((s) => dayjs(s, DATE_FORMAT))
