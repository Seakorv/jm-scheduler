import "./App.css";
import { Button } from "./components/Button/Button";

function App() {
  return (
    <div>
      <h1>Jyväsmetro Aikatauluapp</h1>
      <Button onClick={() => console.log("Böö")}>Testi</Button>
    </div>
  );
}

export default App;
