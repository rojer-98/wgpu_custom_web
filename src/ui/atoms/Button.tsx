import { CSSProperties } from "react";
import Button from "react-bootstrap/Button";

export class ButtonProps {
  id: string;
  inner: string;
  variant?: string;
  style?: CSSProperties;

  handleHover?: () => void;
  handleClick: () => void;

  constructor(
    { id, inner, variant, style, handleClick, handleHover }: {
      id: string;
      inner: string;
      variant?: string;
      style?: CSSProperties;
      handleClick: () => void;
      handleHover?: () => void;
    },
  ) {
    this.id = id;
    this.inner = inner;
    this.variant = variant;
    this.style = style;

    this.handleClick = handleClick;
    this.handleHover = handleHover;
  }

  element() {
    return (
      <div>
        <Button
          id={this.id}
          onClick={this.handleClick}
          onMouseOver={this.handleHover}
          style={this.style}
          variant={this.variant}
        >
          {this.inner}
        </Button>
      </div>
    );
  }
}

export default function MyButton(props: ButtonProps) {
  return props.element();
}
