import { useEffect, useState } from "react";
import { useAuthContext } from "../../utils/auth";
import { makeRequest } from "../../utils/backend";
import "./env_vars.scss";
import { FaCopy, FaCross, FaEye, FaEyeSlash } from "react-icons/fa6";
import { FaEdit, FaRegTimesCircle, FaSave } from "react-icons/fa";

export default function EnvVars({ projectName }: { projectName?: string }) {
    const auth = useAuthContext();
    const [envVars, setEnvVars] = useState<{ key: string; value: string }[]>(
        [],
    );
    const [message, setMessage] = useState<string>("");
    const [loaded, setLoaded] = useState<boolean>(false);
    const [updates, setUpdates] = useState<Map<string, string>>(new Map());

    const fetchEnvVars = async () => {
        if (!projectName) {
            setMessage("Project name not found.");
            return;
        }
        setMessage("Fetching project details...");
        const resp = await makeRequest(
            `${projectName}/get_env`,
            "post",
            null,
            auth.jwt,
        );

        if (resp.status == "success") {
            setEnvVars(
                Object.keys(resp.data).map((key) => ({
                    key,
                    value: resp.data[key],
                })),
            );
            setMessage("");
            setLoaded(true);
        } else {
            setMessage(
                `Error fetching environment variables (${resp.status_code}): ${resp.message}`,
            );
        }
    };

    useEffect(() => {
        if (auth.isAuthenticated) {
            fetchEnvVars();
        }
    }, []);

    const saveEnvVars = async () => {
        //     if (!projectName) {
        //         setMessage("Project name not found.");
        //         return;
        //     }
        //     // check if any changes
        //     if (editingCopy.every((envVar, i) => envVar.value === envVars[i].value)) {
        //         setEditing(false);
        //         return;
        //     }
        //     const confirm = window.confirm(
        //         "Are you sure you want to save the changes to the environment variables?",
        //     );
        //     if (!confirm) return;
        //     const envVarsMap: Record<string, string> = {};
        //     editingCopy.forEach((envVar, i) => {
        //         if (envVar.value !== envVars[i].value) { // only save if the value has changed
        //             envVarsMap[envVar.key] = envVar.value;
        //         }
        //     });
        //     setMessage("Saving environment variables...");
        //     const resp = await makeRequest(
        //         `${projectName}/update_env`,
        //         "post",
        //         envVarsMap,
        //         auth.jwt,
        //     );
        //     if (resp.status == "success") {
        //         setEnvVars(editingCopy
        //             .map((envVar) => ({
        //                 key: envVar.key,
        //                 value: envVar.value,
        //             }))
        //         );
        //         setEditing(false);
        //         setMessage("Restarting deployment to apply changes...");
        //         const restartResp = await makeRequest(
        //             `${projectName}/restart`,
        //             "post",
        //             null,
        //             auth.jwt,
        //         );
        //         if (restartResp.status == "success") {
        //             setMessage("Environment variables updated and deployment restarted successfully.");
        //         } else {
        //             setMessage(
        //                 `Environment variables updated, but error restarting deployment (${restartResp.status_code}): ${restartResp.message}`,
        //             );
        //         }
        //     } else {
        //         setMessage(
        //             `Error saving environment variables (${resp.status_code}): ${resp.message}`,
        //         );
        //     }
    };

    const updateEnvVar = (key: string, value: string) => {
        const newUpdates = new Map(updates);
        if (value == envVars.find((envVar) => envVar.key === key)?.value) {
            if (updates.has(key)) newUpdates.delete(key);
        } else {
            newUpdates.set(key, value);
        }
        setUpdates(newUpdates);
    };

    const editing = updates.size > 0;

    return (
        <div className="env-vars-container">
            <div className="header">
                <h2>Project Environment Variables</h2>
                <div className="actions">
                    {editing && (
                        <>
                            <button
                                className="save-button"
                                onClick={saveEnvVars}
                            >
                                Save
                            </button>
                        </>
                    )}
                </div>
            </div>
            {message && <p>{message}</p>}
            {loaded && (
                <table className="env-vars-table">
                    <colgroup>
                        <col style={{ width: "auto" }} />
                        <col style={{ width: "auto" }} />
                        <col style={{ width: "160px" }} />
                    </colgroup>
                    <thead>
                        <tr>
                            <th>Key</th>
                            <th>Value</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {envVars.map((envVar, i) => (
                            <EnvVarRow key={envVar.key} envVar={envVar} update={updateEnvVar} />
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    );
}

function EnvVarRow({
    envVar,
    update,
}: {
    envVar: { key: string; value: string };
    update: (key: string, value: string) => void;
}) {
    const [visible, setVisible] = useState<boolean>(false);
    const [value, setValue] = useState<string>(envVar.value);
    const [editing, setEditing] = useState<boolean>(false);

    const updated = value !== envVar.value;

    return (
        <tr key={envVar.key} className={updated ? "updated" : ""}>
            <td>{envVar.key}</td>
            <td>
                {editing ? (
                    <input
                        value={value}
                        onChange={(e) => {
                            setValue(e.target.value);
                            update(envVar.key, e.target.value);
                        }}
                    />
                ) : (
                    <span>{visible ? envVar.value : "********"}</span>
                )}
            </td>
            <td>
                <button
                    className="icon-button"
                    onClick={() => {
                        if (editing) {
                            setEditing(false);
                            setValue(envVar.value);
                            update(envVar.key, envVar.value);
                        } else {
                            setEditing(true);
                        }
                    }}
                >
                    {!editing ? (
                        <FaEdit size={16} />
                    ) : (
                        <FaRegTimesCircle size={16} />
                    )}
                </button>
                <button
                    className="icon-button"
                    onClick={() => setVisible(!visible)}
                >
                    {visible ? <FaEyeSlash size={16} /> : <FaEye size={16} />}
                </button>
                <button
                    className="icon-button"
                    onClick={() => {
                        navigator.clipboard.writeText(envVar.value);
                    }}
                >
                    <FaCopy size={16} />
                </button>
            </td>
        </tr>
    );
}
