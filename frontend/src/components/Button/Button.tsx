import styles from "./Button.module.css";
import clsx from "clsx";

type ButtonProps = React.ComponentProps<"button">;

export function Button({ className, ...props }: ButtonProps) {
  return <button className={clsx(styles.button, className)} {...props} />;
}
