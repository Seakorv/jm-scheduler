import "./App.css";
import fi from "@shared/i18n/fi.json";
import { Button } from "./components/Button/Button";

function App() {
  return (
    <div>
      <h1>{fi.app.title}</h1>
      <Button onClick={() => console.log("Böö")}>Testi</Button>
    </div>
  );
}

export default App;
