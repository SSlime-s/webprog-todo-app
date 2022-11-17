import { z } from 'zod'
import dayjs from 'dayjs'

// YYYY-mm-dd HH:MM:SS
export const dateSchema = z
  .string()
  .refine((s) => dayjs(s, 'YYYY-MM-DD HH:mm:ss').isValid())
  .transform((s) => dayjs(s, 'YYYY-MM-DD HH:mm:ss'))
