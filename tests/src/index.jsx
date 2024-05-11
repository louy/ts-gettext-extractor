// To update snapshots, run:
// cargo run -- --path ./tests/src/ --output-folder ./tests/expected-output/

export const App =React.memo(() => {
  const number = useMemo(() => Math.round(10 * Math.random()), []);
  return (
    <>
    <header>
      <h1>{/* A comment to extract */ __p('title', 'App')}</h1>
    </header>
    <main>
      <p>{__('Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur?')}</p>

      <p>{__d('draw', 'Today is your lucky day!')}</p>
      <p>{__dp('draw', 'You won 1 coffee.', 'You won %d coffees.', number)}</p>
    </main>
    </>
  )
});
