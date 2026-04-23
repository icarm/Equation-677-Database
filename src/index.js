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
      <section class="question">
        <p class="lede">Can we find a finite magma <em>M</em> that satisfies</p>
        <div class="eq-line">
          <span class="eq">&forall; x y : M, &nbsp; x = y &#9671; (x &#9671; ((y &#9671; x) &#9671; y))</span>
          <span class="eq-label">(<a href="https://teorth.github.io/equational_theories/implications/?677">Equation 677</a>)</span>
        </div>
        <p class="lede">but does <strong>not</strong> satisfy</p>
        <div class="eq-line">
          <span class="eq">&forall; x : M, &nbsp; x = ((x &#9671; x) &#9671; x) &#9671; x &thinsp;?</span>
          <span class="eq-label">(<a href="https://teorth.github.io/equational_theories/implications/?255">Equation 255</a>)</span>
        </div>
      </section>
      <ul>
        <li><a href="https://github.com/memoryleak47/eq677">github.com/memoryleak47/eq677</a></li>
        <li><a href="https://teorth.github.io/equational_theories/">Equational Theories Project</a></li>
      </ul>
    </main>
    <footer><a href="https://icarm.io">icarm.io</a></footer>
  </body>
</html>
`),
)

export default app
