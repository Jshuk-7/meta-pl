
                            if let Some(pos) = self.variables.iter().position(|v| v.var.name == var.var.name) {
                                self.variables.remove(pos);
                                self.variables.insert(pos, var.clone());
                            }