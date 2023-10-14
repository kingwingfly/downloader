import HomeBtn from "@/components/homeBtn";
import NewTaskBar from "@/components/newTaskBar";

export default function newTask() {
    return (
        <div className="flex">
            <NewTaskBar />
            <HomeBtn />
        </div>
    )
}