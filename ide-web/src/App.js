import logo from './logo.svg';
import './App.css';
import Editor from '@monaco-editor/react';

function App() {
  return (
    <div className="App">
      <Editor height='100vh' defaultLanguage='typescript' theme='vs-dark'/>
    </div>
  );
}

export default App;
