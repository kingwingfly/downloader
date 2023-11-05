import BtnLink from "@/components/btn-link";
import ConfigIcon from "@/components/config-icon";

export default function Home() {
  return (
    <>
      <BtnLink href="\newTask" content="New a Task" scoll={false} />
      <BtnLink href="\taskList" content="Task List" scoll={false} />
      <ConfigIcon />
    </>
  )
}