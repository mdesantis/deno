import { renderToString } from "./deps.ts"

interface Props {
  content: string
}

function DemoReactComponent(props: Props) {
  return <div>{props.content}</div>
}

function renderDemoReactComponentToString(props: Props): string {
  const string = renderToString(<DemoReactComponent {...props}/>)
  return string
}

declare global {
  namespace globalThis {
    // deno_lint bug: https://github.com/denoland/deno_lint/issues/1363
    // deno-lint-ignore no-var
    var test: (props: Props) => string
  }
}

globalThis.test = renderDemoReactComponentToString
