import { Container } from '@cloudflare/containers'

export class Canonicalizer extends Container {
  defaultPort = 8080
  sleepAfter = '10m'
}
