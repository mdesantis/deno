import { renderToString } from "./deps.ts";

interface Props {
  content: string
}

function DemoReactComponent(props: Props) {
  return (
    <div>
      {props.content}
    </div>
  )
}

function renderDemoReactComponentToString(props: Props) {
  return renderToString(<DemoReactComponent {...props}/>)
}

globalThis.test = renderDemoReactComponentToString;
