import { Header } from "./components/Header";
import { Navigation } from "./components/Navigation";
import { Outlet } from "react-router-dom";

export const Layout = () => {
  return (
    <>
      <Header />
      <Outlet />
      <Navigation />
    </>
  );
};
