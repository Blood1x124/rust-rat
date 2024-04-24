import React, { useContext, useEffect, useState, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { RATContext } from "../rat/RATContext";
import { ContextMenuProps, SubMenuProps } from "../../types";
import { manageClientCmd, handleSystemCommandCmd } from "../rat/RATCommands";

const menuOptions = [
  {
    label: "Dashboard",
    icon: <i className="ri-dashboard-line ri-2x text-primary"></i>,
    navigate: true,
    path: "/clients/[ID]",
  },
  {
    label: "Manage",
    icon: <i className="ri-settings-3-line ri-2x text-accent"></i>,
    options: [
      {
        label: "File Manager",
        icon: <i className="ri-folder-line ri-2x text-warning"></i>,
        navigate: true,
        path: "/clients/[ID]/files",
      },
      {
        label: "Revere Shell",
        icon: <i className="ri-terminal-box-line ri-2x text-error"></i>,
        navigate: true,
        path: "/clients/[ID]/shell",
      },
      {
        label: "Process Viewer",
        icon: <i className="ri-cpu-line ri-2x text-info"></i>,
        navigate: true,
        path: "/clients/[ID]/process",
      },
    ],
    navigate: false,
  },
  {
    label: "System",
    icon: <i className="ri-computer-line ri-2x text-info"></i>,
    options: [
      {
        label: "Shutdown",
        icon: <i className="ri-shut-down-line ri-2x text-error"></i>,
        run: "shutdown",
        function: handleSystemCommandCmd,
      },
      {
        label: "Reboot",
        icon: <i className="ri-restart-line ri-2x text-warning"></i>,
        run: "reboot",
        function: handleSystemCommandCmd,
      },
      {
        label: "Log Out",
        icon: <i className="ri-lock-line ri-2x text-info"></i>,
        run: "logout",
        function: handleSystemCommandCmd,
      },
    ],
    navigate: false,
  },
  {
    label: "Connection",
    icon: <i className="ri-link ri-2x text-base-content"></i>,
    options: [
      {
        label: "Reconnect",
        icon: <i className="ri-triangle-line ri-2x text-success"></i>,
        run: "reconnect",
        function: manageClientCmd,
      },
      {
        label: "Disconnect",
        icon: <i className="ri-pentagon-line ri-2x text-error"></i>,
        run: "reconnect",
        function: manageClientCmd,
      },
    ],
  },
];

const SubMenu: React.FC<SubMenuProps> = ({
  items,
  top,
  left,
  id,
  navigate,
  onClose,
}) => {
  return (
    <div
      style={{ top: `${top}px`, left: `${left + 2}px` }}
      className="fixed shadow-lg border border-base-content rounded-md list-none flex flex-col text-center bg-base-200"
    >
      {items.map((item, index) => (
        <div
          key={index}
          onClick={() => {
            if (item.navigate && typeof item.path === "string") {
              navigate(item.path.replace("[ID]", id));
            }
            if (
              typeof item.function == "function" &&
              typeof item.run === "string"
            ) {
              item.function(String(id), item.run);
            }
            onClose();
          }}
          className="btn no-animation justify-normal w-full"
        >
          {item.icon}
          {item.label}
        </div>
      ))}
    </div>
  );
};

export const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  id,
  onClose,
  clientFullName,
}) => {
  const { setSelectedClient } = useContext(RATContext)!;
  const navigate = useNavigate();
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const [submenuPosition, setSubmenuPosition] = useState({ top: 0, left: 0 });
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  const [menuActive, setMenuActive] = useState(false);

  useEffect(() => {
    setSelectedClient(clientFullName);
  }, [clientFullName]);

  const handleMouseEnter = (
    index: number,
    event: React.MouseEvent<HTMLDivElement, MouseEvent>
  ) => {
    setMenuActive(true);
    setActiveIndex(index);
    const rect = event.currentTarget.getBoundingClientRect();
    setSubmenuPosition({ top: rect.top, left: rect.right });
  };

  const handleMouseLeave = () => {
    if (menuActive) return;
    setActiveIndex(null);
  };

  return (
    <div
      className={`fixed context-menu top-[${y}px] left-[${x}px] bg-base-200 border border-base-content rounded-md list-none flex flex-col text-center`}
      style={{
        top: `${y}px`,
        left: `${x}px`,
      }}
    >
      {menuOptions.map((option, index) => (
        <div
          key={index}
          ref={(el) => (itemRefs.current[index] = el)}
          className="btn no-animation flex justify-normal w-full"
          onMouseEnter={(e) => handleMouseEnter(index, e)}
          onMouseLeave={handleMouseLeave}
          onClick={() => {
            if (option.navigate && typeof option.path === "string") {
              navigate(option.path.replace("[ID]", id));
            }
            onClose();
          }}
        >
          {option.icon}
          {option.label}
          {!option.navigate && option.options && (
            <i className="ri-arrow-right-line ml-auto"></i>
          )}
        </div>
      ))}
      {activeIndex !== null && menuOptions[activeIndex].options && (
        <SubMenu
          items={menuOptions[activeIndex].options!}
          top={submenuPosition.top}
          left={submenuPosition.left}
          id={id}
          navigate={navigate}
          onClose={onClose}
        />
      )}
    </div>
  );
};
