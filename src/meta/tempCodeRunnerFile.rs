for arg in args.iter() {
                    arguments
                        .write_fmt(format_args!("\t{}: {},\n", arg.name, arg._type))
                        .unwrap();
                }