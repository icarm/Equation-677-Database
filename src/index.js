import { Hono } from 'hono'

const app = new Hono()

app.get('/', (c) =>
  c.html(`<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Equation 677 Database</title>
    <link rel="stylesheet" href="/style.css" />
  </head>
  <body>
    <header>
      <div class="inner">
        <h1>Equation 677 Database</h1>
        <nav>
          <a href="/">Home</a>
        </nav>
      </div>
    </header>
    <main>
      <h2>Hello, world.</h2>
      <p>Welcome to the Equation 677 Database.</p>
    </main>
    <footer><a href="https://icarm.io">icarm.io</a></footer>
  </body>
</html>
`),
)

export default app
