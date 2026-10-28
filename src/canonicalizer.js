import { Container } from '@cloudflare/containers'

export class Canonicalizer extends Container {
  defaultPort = 8080
  // The instance is billed for its provisioned memory the whole time it is
  // awake, so let it sleep soon after a burst of submissions; a cold start
  // costs only a few seconds.
  sleepAfter = '2m'
}
