import { HashRouter, Route, Routes } from "react-router-dom"
import { AuthProvider } from "./utils/auth"
import OAuthPage from "./pages/OAuthPage"
import MainPage from "./pages/MainPage"
import ProjectPage from "./pages/ProjectPage"

function App() {
  return (
    <>
      <HashRouter>
        <AuthProvider>
          <Routes>
            <Route
              path="/"
              element={<MainPage />}
            />
            <Route
              path="/oauth"
              element={<OAuthPage />}
            />
            <Route
              path="/p/:projectName"
              element={<ProjectPage />}
            />
          </Routes>
        </AuthProvider>
      </HashRouter>
    </>
  )
}

export default App
