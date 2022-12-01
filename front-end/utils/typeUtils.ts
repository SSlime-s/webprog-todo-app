export const unreachable = (x: never): never => {
  throw new Error('unreachable')
  return x
}
