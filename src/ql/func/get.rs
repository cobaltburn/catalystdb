use crate::{
    dbs::{graph::Graph, ops::retrieve::Retrieve},
    doc::document::Cursor,
    err::Error,
    ql::{
        array::Array,
        part::{Next, Part, Skip},
        value::Value,
    },
    resp::Response,
};
use actix::Addr;
use reblessive::tree::Stk;

impl Value {
    pub async fn get(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
        path: &[Part],
    ) -> Result<Value, Error> {
        match path.first() {
            Some(p) => match self {
                Value::Object(_v) => todo!(),
                Value::Edge(_v) => todo!(),
                Value::Record(v) => {
                    let val = v.clone();
                    if path.len() == 0 {
                        return Ok(Value::Record(val));
                    }
                    match p {
                        Part::Path(_s) => todo!(),
                        _ => {
                            let response = graph.send(Retrieve::Record(*val)).await.unwrap();
                            let Response::Record(node) = response else {
                                return Ok(Value::None);
                            };
                            todo!()
                        }
                    }
                }
                Value::Array(v) => match p {
                    Part::All | Part::Flatten => {
                        let path = path.next();
                        let mut mapped = Array::with_capacity(v.len());
                        for v in v.iter() {
                            let val = stk.run(|stk| v.get(stk, graph, cur, path)).await?;
                            mapped.push(val);
                        }
                        Ok(Value::Array(mapped))
                    }
                    Part::First => match v.first() {
                        Some(v) => {
                            stk.run(|stk| stk.run(|stk| v.get(stk, graph, cur, path.next())))
                                .await
                        }
                        None => {
                            stk.run(|stk| Value::None.get(stk, graph, cur, path.next()))
                                .await
                        }
                    },
                    Part::Last => match v.last() {
                        Some(v) => stk.run(|stk| v.get(stk, graph, cur, path.next())).await,
                        None => {
                            stk.run(|stk| Value::None.get(stk, graph, cur, path.next()))
                                .await
                        }
                    },
                    Part::Index(i) => match v.get(i.to_usize()) {
                        Some(v) => stk.run(|stk| v.get(stk, graph, cur, path.next())).await,
                        None => {
                            stk.run(|stk| Value::None.get(stk, graph, cur, path.next()))
                                .await
                        }
                    },
                    Part::Where(w) => {
                        let mut array = Array::with_capacity(v.len());
                        for v in v.iter() {
                            let cur = v.clone().into();
                            if stk
                                .run(|stk| w.evaluate(stk, graph, Some(&cur)))
                                .await?
                                .is_truthy()
                            {
                                array.push(v.clone());
                            }
                        }
                        let v = Value::Array(array);
                        stk.run(|stk| v.get(stk, graph, cur, path.next())).await
                    }
                    Part::Value(x) => match stk.run(|stk| x.evaluate(stk, graph, cur)).await? {
                        Value::Number(i) => match v.get(i.to_usize()) {
                            Some(v) => stk.run(|stk| v.get(stk, graph, cur, path.next())).await,
                            None => Ok(Value::None),
                        },
                        _ => {
                            stk.run(|stk| Value::None.get(stk, graph, cur, path.next()))
                                .await
                        }
                    },
                    _ => {
                        let len = match path.get(1) {
                            Some(Part::All) => 2,
                            _ => 1,
                        };

                        let mut mapped = Array::with_capacity(v.len());
                        for v in v.iter() {
                            let val = stk.run(|stk| v.get(stk, graph, cur, &path[0..len])).await?;
                            mapped.push(val);
                        }
                        let mapped = Value::Array(mapped);

                        let mapped = match (path.first(), path.get(1)) {
                            (Some(Part::Path(_)), Some(Part::Path(_))) => mapped.flatten(),
                            (Some(Part::Path(_)), Some(Part::Where(_))) => mapped.flatten(),
                            _ => mapped,
                        };
                        stk.run(|stk| mapped.get(stk, graph, cur, path.skip(len)))
                            .await
                    }
                },
                v => Ok(v.clone()),
            },
            None => Ok(self.clone()),
        }
    }
}
