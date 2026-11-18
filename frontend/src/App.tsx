import "./App.css";
import { useTranslation } from "react-i18next";
import { Button } from "./components/Button/Button";

function App() {
  const { t } = useTranslation();

  return (
    <div>
      <h1>{t("app.title")}</h1>
      <Button onClick={() => console.log("Böö")}>Testi</Button>
    </div>
  );
}

export default App;
