import { BrowserRouter, Route, Routes } from "react-router-dom"
import { AuthProvider } from "./utils/auth"
import OAuthPage from "./pages/OAuthPage"
import MainPage from "./pages/MainPage"

function App() {
  return (
    <>
      <BrowserRouter>
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
          </Routes>
        </AuthProvider>
      </BrowserRouter>
    </>
  )
}

export default App
