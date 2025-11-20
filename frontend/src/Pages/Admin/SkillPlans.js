import React from "react";
import { apiCall, toaster } from "../../api";
import { AuthContext, ToastContext } from "../../contexts";
import { Content, PageTitle } from "../../Components/Page";
import { Table, Row, Cell, TableHead, TableBody, CellHead } from "../../Components/Table";
import { Button, Buttons, Input, Select } from "../../Components/Form";
import { usePageTitle } from "../../Util/title";
import { Modal } from "../../Components/Modal";

export function SkillPlans() {
  const authContext = React.useContext(AuthContext);
  if (!authContext) {
    return (
      <Content>
        <b>Login Required!</b>
      </Content>
    );
  }
  return <SkillPlansAdmin authContext={authContext} />;
}

function SkillPlansAdmin({ authContext }) {
  const toastContext = React.useContext(ToastContext);
  const [plans, setPlans] = React.useState(null);
  const [editingPlan, setEditingPlan] = React.useState(null);
  const [isNewPlan, setIsNewPlan] = React.useState(false);
  const [modalOpen, setModalOpen] = React.useState(false);
  const [rawYaml, setRawYaml] = React.useState(null);
  const [rawYamlModalOpen, setRawYamlModalOpen] = React.useState(false);

  usePageTitle("Admin - Skill Plans");

  const loadPlans = React.useCallback(async () => {
    try {
      const data = await apiCall("/api/admin/skillplans", {});
      setPlans(data);
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  }, [toastContext]);

  React.useEffect(() => {
    loadPlans();
  }, [loadPlans]);

  const loadRawYaml = async () => {
    try {
      const yaml = await apiCall("/api/admin/skillplans/raw", {});
      setRawYaml(yaml);
      setRawYamlModalOpen(true);
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const saveRawYaml = async () => {
    try {
      // Use fetch directly since apiCall expects JSON
      const response = await fetch("/api/admin/skillplans/raw", {
        method: "POST",
        headers: {
          "Content-Type": "text/yaml",
        },
        credentials: "include",
        body: rawYaml,
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `HTTP ${response.status}`);
      }

      const result = await response.text();
      toaster(toastContext, Promise.resolve(result || "Raw YAML saved successfully"));
      setRawYamlModalOpen(false);
      loadPlans(); // Reload plans to reflect changes
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const handleEdit = (plan) => {
    setEditingPlan(JSON.parse(JSON.stringify(plan))); // Deep clone
    setIsNewPlan(false);
    setModalOpen(true);
  };

  const handleNew = () => {
    setEditingPlan({
      name: "",
      description: "",
      plan: [],
    });
    setIsNewPlan(true);
    setModalOpen(true);
  };

  const handleDelete = async (planName) => {
    if (!window.confirm(`Are you sure you want to delete "${planName}"?`)) {
      return;
    }

    try {
      await apiCall(`/api/admin/skillplans/${encodeURIComponent(planName)}`, {
        method: "DELETE",
      });
      toaster(toastContext, Promise.resolve(`Plan "${planName}" deleted successfully`));
      loadPlans();
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const handleSave = async () => {
    try {
      const url = isNewPlan
        ? "/api/admin/skillplans"
        : `/api/admin/skillplans/${encodeURIComponent(editingPlan.name)}`;
      const method = isNewPlan ? "POST" : "PUT";

      await apiCall(url, {
        method,
        json: { plan: editingPlan },
      });

      toaster(
        toastContext,
        Promise.resolve(`Plan "${editingPlan.name}" ${isNewPlan ? "created" : "updated"} successfully`)
      );
      setModalOpen(false);
      loadPlans();
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const addPlanStep = () => {
    setEditingPlan({
      ...editingPlan,
      plan: [...editingPlan.plan, { type: "tank", from: "starter" }],
    });
  };

  const removePlanStep = (index) => {
    setEditingPlan({
      ...editingPlan,
      plan: editingPlan.plan.filter((_, i) => i !== index),
    });
  };

  const getStepType = (step) => {
    // Backend uses serde tag = "type", so the type field exists directly
    if (step.type) return step.type;
    // Fallback: determine from structure
    if ("hull" in step && "fit" in step) return "fit";
    if ("from" in step && "tier" in step) return "skills";
    if ("from" in step && "level" in step) return "skill";
    if ("from" in step && !("tier" in step) && !("level" in step)) return "tank";
    return "tank";
  };

  const updatePlanStep = (index, field, value) => {
    const newPlan = [...editingPlan.plan];
    if (field === "type") {
      // Reset step when type changes
      if (value === "fit") {
        newPlan[index] = { type: "fit", hull: "", fit: "" };
      } else if (value === "skills") {
        newPlan[index] = { type: "skills", from: "", tier: "min" };
      } else if (value === "skill") {
        newPlan[index] = { type: "skill", from: "", level: 1 };
      } else if (value === "tank") {
        newPlan[index] = { type: "tank", from: "starter" };
      }
    } else {
      newPlan[index] = { ...newPlan[index], [field]: value };
    }
    setEditingPlan({ ...editingPlan, plan: newPlan });
  };

  if (!plans) {
    return <Content>Loading...</Content>;
  }

  return (
    <>
      <PageTitle>Admin - Skill Plans</PageTitle>
      <Content>
        <Buttons style={{ marginBottom: "1em" }}>
          <Button onClick={handleNew}>Add New Plan</Button>
          <Button onClick={loadRawYaml}>Edit Raw YAML</Button>
        </Buttons>

        <Table fullWidth>
          <TableHead>
            <Row>
              <CellHead>Name</CellHead>
              <CellHead>Description</CellHead>
              <CellHead>Steps</CellHead>
              <CellHead style={{ width: "150px" }}>Actions</CellHead>
            </Row>
          </TableHead>
          <TableBody>
            {plans.map((plan) => (
              <Row key={plan.name}>
                <Cell>{plan.name}</Cell>
                <Cell>{plan.description}</Cell>
                <Cell>{plan.plan.length}</Cell>
                <Cell>
                  <Buttons marginb={"0em"}>
                    <Button onClick={() => handleEdit(plan)}>Edit</Button>
                    <Button onClick={() => handleDelete(plan.name)}>Delete</Button>
                  </Buttons>
                </Cell>
              </Row>
            ))}
          </TableBody>
        </Table>
      </Content>

      <Modal open={modalOpen} setOpen={setModalOpen}>
        {editingPlan && (
          <div style={{ minWidth: "600px", maxWidth: "90vw" }}>
            <h2>{isNewPlan ? "New" : "Edit"} Skill Plan</h2>

            <div style={{ marginBottom: "1em" }}>
              <label>
                <strong>Name:</strong>
                <br />
                <Input
                  value={editingPlan.name}
                  onChange={(e) => setEditingPlan({ ...editingPlan, name: e.target.value })}
                  disabled={!isNewPlan}
                />
              </label>
            </div>

            <div style={{ marginBottom: "1em" }}>
              <label>
                <strong>Description:</strong>
                <br />
                <textarea
                  value={editingPlan.description}
                  onChange={(e) => setEditingPlan({ ...editingPlan, description: e.target.value })}
                  style={{ width: "100%", minHeight: "80px", padding: "0.5em" }}
                />
              </label>
            </div>

            <div style={{ marginBottom: "1em" }}>
              <strong>Plan Steps:</strong>
              <Button onClick={addPlanStep} style={{ marginLeft: "1em" }}>
                Add Step
              </Button>
            </div>

            {editingPlan.plan.map((step, index) => (
              <div
                key={index}
                style={{
                  border: "1px solid #ccc",
                  padding: "1em",
                  marginBottom: "1em",
                  borderRadius: "4px",
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "0.5em" }}>
                  <strong>Step {index + 1}</strong>
                  <Button onClick={() => removePlanStep(index)}>Remove</Button>
                </div>

                <div style={{ marginBottom: "0.5em" }}>
                  <label>
                    Type:
                    <Select
                      value={getStepType(step)}
                      onChange={(e) => updatePlanStep(index, "type", e.target.value)}
                    >
                      <option value="fit">Fit</option>
                      <option value="skills">Skills</option>
                      <option value="skill">Skill</option>
                      <option value="tank">Tank</option>
                    </Select>
                  </label>
                </div>

                {getStepType(step) === "fit" && (
                  <>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        Hull:
                        <Input
                          value={step.hull || ""}
                          onChange={(e) => updatePlanStep(index, "hull", e.target.value)}
                        />
                      </label>
                    </div>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        Fit:
                        <Input
                          value={step.fit || ""}
                          onChange={(e) => updatePlanStep(index, "fit", e.target.value)}
                        />
                      </label>
                    </div>
                  </>
                )}

                {getStepType(step) === "skills" && (
                  <>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        From (Ship/Skill Set):
                        <Input
                          value={step.from || ""}
                          onChange={(e) => updatePlanStep(index, "from", e.target.value)}
                        />
                      </label>
                    </div>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        Tier:
                        <Select
                          value={step.tier || "min"}
                          onChange={(e) => updatePlanStep(index, "tier", e.target.value)}
                        >
                          <option value="min">Min</option>
                          <option value="elite">Elite</option>
                          <option value="gold">Gold</option>
                        </Select>
                      </label>
                    </div>
                  </>
                )}

                {getStepType(step) === "skill" && (
                  <>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        Skill Name:
                        <Input
                          value={step.from || ""}
                          onChange={(e) => updatePlanStep(index, "from", e.target.value)}
                        />
                      </label>
                    </div>
                    <div style={{ marginBottom: "0.5em" }}>
                      <label>
                        Level:
                        <Select
                          value={step.level || 1}
                          onChange={(e) => updatePlanStep(index, "level", parseInt(e.target.value))}
                        >
                          <option value={1}>I</option>
                          <option value={2}>II</option>
                          <option value={3}>III</option>
                          <option value={4}>IV</option>
                          <option value={5}>V</option>
                        </Select>
                      </label>
                    </div>
                  </>
                )}

                {getStepType(step) === "tank" && (
                  <div style={{ marginBottom: "0.5em" }}>
                    <label>
                      Level:
                      <Select
                        value={step.from || "starter"}
                        onChange={(e) => updatePlanStep(index, "from", e.target.value)}
                      >
                        <option value="starter">Starter</option>
                        <option value="min">Min</option>
                        <option value="elite">Elite</option>
                        <option value="gold">Gold</option>
                        <option value="bastion">Bastion</option>
                      </Select>
                    </label>
                  </div>
                )}
              </div>
            ))}

            <Buttons>
              <Button onClick={handleSave}>Save</Button>
              <Button onClick={() => setModalOpen(false)}>Cancel</Button>
            </Buttons>
          </div>
        )}
      </Modal>

      <Modal open={rawYamlModalOpen} setOpen={setRawYamlModalOpen}>
        <div style={{ minWidth: "800px", maxWidth: "90vw", maxHeight: "90vh", display: "flex", flexDirection: "column" }}>
          <h2>Edit Raw YAML</h2>
          <div style={{ flex: 1, marginBottom: "1em", minHeight: "500px" }}>
            <textarea
              value={rawYaml || ""}
              onChange={(e) => setRawYaml(e.target.value)}
              style={{
                width: "100%",
                height: "100%",
                minHeight: "500px",
                padding: "0.5em",
                fontFamily: "monospace",
                fontSize: "0.9em",
                whiteSpace: "pre",
                overflowWrap: "normal",
                overflowX: "auto",
              }}
              spellCheck={false}
            />
          </div>
          <Buttons>
            <Button onClick={saveRawYaml}>Save</Button>
            <Button onClick={() => setRawYamlModalOpen(false)}>Cancel</Button>
          </Buttons>
        </div>
      </Modal>
    </>
  );
}

